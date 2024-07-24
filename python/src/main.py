#!/usr/bin/env python
# pylint: disable=unused-argument
# This program is dedicated to the public domain under the CC0 license.

"""
First, a few callback functions are defined. Then, those functions are passed to
the Application and registered at their respective places.
Then, the bot is started and runs until we press Ctrl-C on the command line.

Usage:
Example of a bot-user conversation using nested ConversationHandlers.
Send /start to initiate the conversation.
Press Ctrl-C on the command line or send a signal to the process to stop the
bot.
"""

import logging
from typing import Any, Dict, Tuple
import json

from telegram import InlineKeyboardButton, InlineKeyboardMarkup, Update, User
from telegram.ext import (
    Application,
    CallbackQueryHandler,
    CommandHandler,
    ContextTypes,
    ConversationHandler,
    MessageHandler,
    filters,
    ChatMemberHandler,
)
import connect_info

# Enable logging
logging.basicConfig(
    format="%(asctime)s - %(name)s - %(levelname)s - %(message)s", level=logging.INFO
)
# set higher logging level for httpx to avoid all GET and POST requests being logged
logging.getLogger("httpx").setLevel(logging.WARNING)

logger = logging.getLogger(__name__)

# State definitions for top level conversation
SELECTING_ACTION, USEFUL_LINKS, NOTICE, SELECTING_GROUP = map(chr, range(4))
# State definitions for second level conversation
SELECTING_GROUP, SELECTING_LINK, SELECTING_WIKI_LINK, TKZ_WEB, SA_LAB_WEB, REDMINE_WEB, WIKI_MENU = map(chr, range(4, 11))
# State definitions for descriptions conversation
SELECTING_FEATURE, TYPING = map(chr, range(11, 13))
# Meta states
STOPPING, SHOWING = map(chr, range(13, 15))
# Shortcut for ConversationHandler.END
END = ConversationHandler.END

# Different constants for this example
# (
PARENTS = 'parents'
CHILDREN = 'children'
SELECTED_GROUP = 'select-group'
SELF = 'self'
NAME = 'name'
START_OVER = 'start-over'
FEATURES = 'features'
CURRENT_GROUP = 'current-group'
CURRENT_FEATURE = 'current-feature'
CURRENT_LEVEL = 'current-level'
SUBSCRIBE = 'subscribe'
SUBSCRIBE_GROUP = 'subscribe-group'
#) = map(chr, range(20, 33))

user: User

# Helper
def _name_switcher(level: str) -> Tuple[str, str]:
    if level == PARENTS:
        return "Father", "Mother"
    return "Brother", "Sister"


# Top level conversation callbacks
async def start(update: Update, context: ContextTypes.DEFAULT_TYPE) -> str:
    """Select an action."""
    user = update.effective_user
    context.user_data[user.id] = {
        'id': user.id
    }
    print(f'start | user.id: {user.id}')
    text = (
        "Select your action from menu. "
        "To abort, simply type /stop."
    )
    buttons = [
        [
            InlineKeyboardButton(text="Links", callback_data=str(USEFUL_LINKS)),
            InlineKeyboardButton(text="Notice", callback_data=str(NOTICE)),
        ],
        [
            InlineKeyboardButton(text="Subscribe", callback_data = SUBSCRIBE),
        ],
        [
            # InlineKeyboardButton(text="Show data", callback_data=str(SHOWING)),
            InlineKeyboardButton(text="Done", callback_data=str(END)),
        ],
    ]
    keyboard = InlineKeyboardMarkup(buttons)
    # If we're starting over we don't need to send a new message
    if context.user_data.get(START_OVER):
        await update.callback_query.answer()
        await update.callback_query.edit_message_text(text=text, reply_markup=keyboard)
    else:
        await update.message.reply_text(
            "Hi, I'm TKZ office Bot."
        )
        await update.message.reply_text(text=text, reply_markup=keyboard)
    context.user_data[START_OVER] = False
    return SELECTING_ACTION


async def show_data(update: Update, context: ContextTypes.DEFAULT_TYPE) -> str:
    """Pretty print gathered data."""
    def pretty_print(data: Dict[str, Any], level: str) -> str:
        people = data.get(level)
        if not people:
            return "\nNo information yet."
        return_str = ""
        if level == SELF:
            for person in data[level]:
                return_str += f"\nName: {person.get(NAME, '-')}, Age: {person.get(AGE, '-')}"
        else:
            male, female = _name_switcher(level)

            for person in data[level]:
                gender = female if person[GENDER] == FEMALE else male
                return_str += (
                    f"\n{gender}: Name: {person.get(NAME, '-')}, Age: {person.get(AGE, '-')}"
                )
        return return_str
    user_data = context.user_data
    text = f"Yourself:{pretty_print(user_data, SELF)}"
    text += f"\n\nParents:{pretty_print(user_data, PARENTS)}"
    text += f"\n\nChildren:{pretty_print(user_data, CHILDREN)}"
    buttons = [[InlineKeyboardButton(text="Back", callback_data=str(END))]]
    keyboard = InlineKeyboardMarkup(buttons)
    await update.callback_query.answer()
    await update.callback_query.edit_message_text(text=text, reply_markup=keyboard)
    user_data[START_OVER] = True
    return SHOWING


async def stop(update: Update, context: ContextTypes.DEFAULT_TYPE) -> int:
    """End Conversation by command."""
    await update.message.reply_text("Okay, bye.")
    return END


async def end(update: Update, context: ContextTypes.DEFAULT_TYPE) -> int:
    """End conversation from InlineKeyboardButton."""
    await update.callback_query.answer()
    text = "See you around!"
    await update.callback_query.edit_message_text(text=text)
    return END


# Second level conversation callbacks
async def select_group(update: Update, context: ContextTypes.DEFAULT_TYPE) -> str:
    text = "Okay, please select a group."
    text = "Select the group to send the message."
    buttons = [
        [
            InlineKeyboardButton(text="ТКЗ СПБ", callback_data = 'TKZ_SPB_GROUP'),
            InlineKeyboardButton(text="Office-group", callback_data = 'TKZ_OFFICE_GROUP'),
        ],
        [
            InlineKeyboardButton(text="Back", callback_data = str(END)),
        ],
    ]
    keyboard = InlineKeyboardMarkup(buttons)
    await update.callback_query.answer()
    await update.callback_query.edit_message_text(text=text, reply_markup=keyboard)
    return SELECTING_GROUP

# Second level conversation callbacks
async def subscribe_group(update: Update, context: ContextTypes.DEFAULT_TYPE) -> str:
    text = "Okay, please select a group."
    text = "Select the group to subscribe."
    buttons = [
        [
            InlineKeyboardButton(text="ТКЗ СПБ", callback_data = 'TKZ_SPB_GROUP'),
            InlineKeyboardButton(text="Office-group", callback_data = 'TKZ_OFFICE_GROUP'),
        ],
        [
            InlineKeyboardButton(text="Back", callback_data = str(END)),
        ],
    ]
    keyboard = InlineKeyboardMarkup(buttons)
    await update.callback_query.answer()
    await update.callback_query.edit_message_text(text=text, reply_markup=keyboard)
    return SUBSCRIBE_GROUP


# Second level conversation callbacks
async def select_link(update: Update, context: ContextTypes.DEFAULT_TYPE) -> str:
    text = "Select the link to follow."
    buttons = [
        [
            InlineKeyboardButton(text="ТКЗ", url = 'http://tkz-cranes.ru/about/'),      # callback_data=str(TKZ_WEB)
            InlineKeyboardButton(text="S&A-Lab", url = 'https://sa-lab.dev/'),          # callback_data=str(SA_LAB_WEB)
        ],
        [
            InlineKeyboardButton(text="Redmine", url = 'http://tkz-erm-01.ad.tkz-cranes.ru/'),       # callback_data=str(REDMINE_WEB)
            InlineKeyboardButton(text="Wiki", callback_data=str(WIKI_MENU)),
        ],
        [
            InlineKeyboardButton(text="...", callback_data=str(END)),
            InlineKeyboardButton(text="Back", callback_data=str(END)),
        ],
    ]
    keyboard = InlineKeyboardMarkup(buttons)
    await update.callback_query.answer()
    await update.callback_query.edit_message_text(text=text, reply_markup=keyboard)
    return SELECTING_LINK


# Second level conversation callbacks
async def select_wiki_link(update: Update, context: ContextTypes.DEFAULT_TYPE) -> str:
    text = "Our Wiki."
    buttons = [
        [
            InlineKeyboardButton(text = 'Wiki home', url = 'http://192.168.120.90:8083/bin/view/Main/'),
            InlineKeyboardButton(text = 'Wiki Прием на работу', url = 'http://192.168.120.90:8083/bin/view/%D0%9E%D0%A0%D0%93%D0%90%D0%9D%D0%98%D0%97%D0%90%D0%A6%D0%98%D0%9E%D0%9D%D0%9D%D0%90%D0%AF/%D0%9F%D0%A0%D0%98%D0%95%D0%9C%20%D0%9D%D0%90%20%D0%A0%D0%90%D0%91%D0%9E%D0%A2%D0%A3/'),
        ],
        [
            InlineKeyboardButton(text="...", callback_data=str(END)),
            InlineKeyboardButton(text="Back", callback_data=str(END)),
        ],
    ]
    keyboard = InlineKeyboardMarkup(buttons)
    await update.callback_query.answer()
    await update.callback_query.edit_message_text(text=text, reply_markup=keyboard)
    return SELECTING_WIKI_LINK


async def end_second_level(update: Update, context: ContextTypes.DEFAULT_TYPE) -> int:
    """Return to top level conversation."""
    context.user_data[START_OVER] = True
    await start(update, context)
    return END


async def ask_for_input(update: Update, context: ContextTypes.DEFAULT_TYPE) -> str:
    """Prompt user to input data for selected feature."""
    context.user_data[CURRENT_FEATURE] = update.callback_query.data
    user = update.effective_user
    context.user_data[user.id]['group'] = update.callback_query.data
    text = "Okay, tell me."
    await update.callback_query.answer()
    await update.callback_query.edit_message_text(text=text)
    return TYPING


async def save_input(update: Update, context: ContextTypes.DEFAULT_TYPE) -> str:
    """Save input for feature and return to feature selection."""
    user = update.effective_user
    context.user_data[user.id]['message'] = update.message.text
    group = context.user_data[user.id]['group']
    message = context.user_data[user.id]['message']
    print(f"save_input | group: {group}")
    print(f"save_input | message: {message}")
    return await notice(update, context)


# Third level callbacks
async def notice(update: Update, context: ContextTypes.DEFAULT_TYPE) -> str:
    """Select a feature to update for the person."""
    user = update.effective_user
    group = context.user_data[user.id]['group']
    message = context.user_data[user.id]['message']
    members = load_members()
    group_title = members[group]['text']
    print(f"notice | message to the '{group_title}': {message}")
    for key in members[group]['members']:
        member = members[group]['members'][key]
        print(f"\t notice | member {member}")
        chat_id = int(member['chat_id'])
        name = member['name']
        print(f"\t notice | sending to the '{name}' ({chat_id})")
        await context.bot.send_message(chat_id=chat_id, text=message)
    return STOPPING



async def end_notice(update: Update, context: ContextTypes.DEFAULT_TYPE) -> int:
    """End sending notice message."""
    # if level == SELF:
    # await start(update, context)
    # else:
    # await select_group(update, context)
    # return END
    return STOPPING


async def stop_nested(update: Update, context: ContextTypes.DEFAULT_TYPE) -> str:
    """Completely end conversation from within nested conversation."""
    await update.message.reply_text("Okay, bye.")
    return STOPPING


async def new_member(update: Update, context: ContextTypes.DEFAULT_TYPE):
    if update.message.new_chat_members:
        memberId = None
        for (member, _) in update.message.new_chat_members:
            if member.username == connect_info.ConnectInfo.botName:
                memberId = member.id
                update.message.reply_text('Welcome')
        print(f"new_member | member id: {memberId}")


# Define a function to handle the messages that the bot receives
async def message_handler(update: Update, context: ContextTypes.DEFAULT_TYPE):
    # Get the message from the update
    message = update.message
    chat_id = update.message.chat_id
    name = update.message.from_user
    print(f'received from: {name} ({chat_id})')
    print(f'received message: {message.text}')


def load_members():
    with open('members.json', encoding='utf-8') as f:
        members = json.load(f)
        return members

def store_members(members):
    with open('members.json', 'w', encoding='utf-8') as f:
        json.dump(members, f, ensure_ascii=False, indent=4)


async def subscribe(update: Update, context: ContextTypes.DEFAULT_TYPE) -> str:
    group = update.callback_query.data
    chat_id = update.effective_user.id
    # phone = update.effective_user.phone_number
    message = update.message
    name = update.effective_user.name
    print(f'subscribe | from: {name} ({chat_id})')
    print(f'subscribe | message: {message}')
    print(f"subscribe | group: {group}")
    members = load_members()
    members[group]['members'][chat_id] = {
        "chat_id": chat_id,
        "name": name
    }
    store_members(members)
    await context.bot.send_message(chat_id, f"You are successfully subscribed to the {group} notifications!")
    print(f'members: {members}')
    return STOPPING


# async def end_subscribe(update: Update, context: ContextTypes.DEFAULT_TYPE) -> int:
#     """Return to top level conversation."""
#     user_id = update.effective_user.id
#     user_name = update.effective_user.id
#     context.user_data[user_id] = None
#     print(f'end_subscribe | Finished for {user_name} ({user_id})')
#     await start(update, context)
#     return END


def main() -> None:
    """Run the bot."""
    # Create the Application and pass it your bot's token.
    application = Application.builder().token(connect_info.ConnectInfo.botToken).build()
    # Set up 'Select Link' level ConversationHandler
    select_link_conv = ConversationHandler(
        entry_points=[CallbackQueryHandler(select_link, pattern="^" + str(USEFUL_LINKS) + "$")],
        states={
            SELECTING_LINK: [
                CallbackQueryHandler(select_wiki_link, pattern=f"^{WIKI_MENU}$"),
            ],
        },
        fallbacks=[
            CallbackQueryHandler(end_second_level, pattern="^" + str(END) + "$"),
            CommandHandler("stop", stop_nested),
        ],
        map_to_parent={
            # After showing data return to top level menu
            SHOWING: SHOWING,
            # Return to top level menu
            END: SELECTING_ACTION,
            # End conversation altogether
            STOPPING: END,
        },
    )
    # Set up Notice level ConversationHandler
    notice_conv = ConversationHandler(
        entry_points=[CallbackQueryHandler(select_group, pattern="^" + str(NOTICE) + "$")],
        states={
            SELECTING_GROUP: [
                CallbackQueryHandler(ask_for_input, pattern="^(?!" + str(END) + ").*$"),
            ],
            TYPING: [
                MessageHandler(filters.TEXT & ~filters.COMMAND, save_input),
            ],
        },
        fallbacks=[
            CallbackQueryHandler(end_second_level, pattern="^" + str(END) + "$"),
            CommandHandler("stop", stop_nested),
        ],
        map_to_parent={
            # Return to second level menu
            END: SELECTING_GROUP,
            # End conversation altogether
            STOPPING: END,
        },
    )
    # Set up Subscribe level ConversationHandler
    subscribe_group_conv = ConversationHandler(
        entry_points=[CallbackQueryHandler(subscribe_group, pattern="^" + str(SUBSCRIBE) + "$")],
        states={
            SUBSCRIBE_GROUP: [
                CallbackQueryHandler(subscribe, pattern="^.+GROUP$"),
            ],
        },
        fallbacks=[
            CallbackQueryHandler(end_second_level, pattern="^" + str(END) + "$"),
            CommandHandler("stop", stop_nested),
        ],
        map_to_parent={
            # Return to top level menu
            END: SUBSCRIBE_GROUP,
            # End conversation altogether
            STOPPING: END,
        },
    )
    # Set up top level ConversationHandler (selecting action)
    # Because the states of the third level conversation map to the ones of the second level
    # conversation, we need to make sure the top level conversation can also handle them
    selection_handlers = [
        select_link_conv,
        notice_conv,
        subscribe_group_conv,
        CallbackQueryHandler(end, pattern="^" + str(END) + "$"),
    ]
    conv_handler = ConversationHandler(
        entry_points=[CommandHandler("start", start)],
        states={
            SHOWING: [CallbackQueryHandler(start, pattern="^" + str(END) + "$")],
            SELECTING_ACTION: selection_handlers,
            SELECTING_GROUP: selection_handlers,
            SUBSCRIBE_GROUP: selection_handlers,
            STOPPING: [CommandHandler("start", start)],
        },
        fallbacks=[CommandHandler("stop", stop)],
    )
    application.add_handler(conv_handler)
    application.add_handler(ChatMemberHandler(new_member))
    application.add_handler(MessageHandler(filters.ALL, message_handler))


    # Run the bot until the user presses Ctrl-C
    application.run_polling(allowed_updates=Update.ALL_TYPES)


if __name__ == "__main__":
    main()